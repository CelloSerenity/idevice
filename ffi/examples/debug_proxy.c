// Jackson Coxson

#include "idevice.h"
#include <arpa/inet.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define MAX_COMMAND_LENGTH 1024

void print_usage(const char *program_name) {
  printf("Usage: %s <device_ip> [pairing_file]\n", program_name);
  printf("Example: %s 10.0.0.1 pairing.plist\n", program_name);
}

int main(int argc, char **argv) {
  // Initialize logger
  idevice_init_logger(Info, Disabled, NULL);

  if (argc < 2) {
    print_usage(argv[0]);
    return 1;
  }

  const char *device_ip = argv[1];
  const char *pairing_file = argc > 2 ? argv[2] : "pairing.plist";

  /*****************************************************************
   * CoreDeviceProxy Setup
   *****************************************************************/
  printf("=== Setting up CoreDeviceProxy ===\n");

  // Create socket address
  struct sockaddr_in addr;
  memset(&addr, 0, sizeof(addr));
  addr.sin_family = AF_INET;
  addr.sin_port = htons(LOCKDOWN_PORT);
  if (inet_pton(AF_INET, device_ip, &addr.sin_addr) != 1) {
    fprintf(stderr, "Invalid IP address\n");
    return 1;
  }

  // Read pairing file
  IdevicePairingFile *pairing = NULL;
  IdeviceErrorCode err = idevice_pairing_file_read(pairing_file, &pairing);
  if (err != IdeviceSuccess) {
    fprintf(stderr, "Failed to read pairing file: %d\n", err);
    return 1;
  }

  // Create TCP provider
  IdeviceProviderHandle *tcp_provider = NULL;
  err = idevice_tcp_provider_new((struct sockaddr *)&addr, pairing,
                                 "DebugProxyShell", &tcp_provider);
  if (err != IdeviceSuccess) {
    fprintf(stderr, "Failed to create TCP provider: %d\n", err);
    idevice_pairing_file_free(pairing);
    return 1;
  }

  // Connect to CoreDeviceProxy
  CoreDeviceProxyHandle *core_device = NULL;
  err = core_device_proxy_connect(tcp_provider, &core_device);
  if (err != IdeviceSuccess) {
    fprintf(stderr, "Failed to connect to CoreDeviceProxy: %d\n", err);
    idevice_provider_free(tcp_provider);
    return 1;
  }
  idevice_provider_free(tcp_provider);

  // Get server RSD port
  uint16_t rsd_port;
  err = core_device_proxy_get_server_rsd_port(core_device, &rsd_port);
  if (err != IdeviceSuccess) {
    fprintf(stderr, "Failed to get server RSD port: %d\n", err);
    core_device_proxy_free(core_device);
    return 1;
  }
  printf("Server RSD Port: %d\n", rsd_port);

  /*****************************************************************
   * Create TCP Tunnel Adapter
   *****************************************************************/
  printf("\n=== Creating TCP Tunnel Adapter ===\n");

  AdapterHandle *adapter = NULL;
  err = core_device_proxy_create_tcp_adapter(core_device, &adapter);
  if (err != IdeviceSuccess) {
    fprintf(stderr, "Failed to create TCP adapter: %d\n", err);
    core_device_proxy_free(core_device);
    return 1;
  }

  // Connect to RSD port
  AdapterStreamHandle *stream = NULL;
  err = adapter_connect(adapter, rsd_port, &stream);
  if (err != IdeviceSuccess) {
    fprintf(stderr, "Failed to connect to RSD port: %d\n", err);
    adapter_free(adapter);
    return 1;
  }
  printf("Successfully connected to RSD port\n");

  /*****************************************************************
   * RSD Handshake
   *****************************************************************/
  printf("\n=== Performing RSD Handshake ===\n");

  RsdHandshakeHandle *handshake = NULL;
  err = rsd_handshake_new(stream, &handshake);
  if (err != IdeviceSuccess) {
    fprintf(stderr, "Failed to perform RSD handshake: %d\n", err);
    adapter_close(stream);
    adapter_free(adapter);
    return 1;
  }

  /*****************************************************************
   * Debug Proxy Setup
   *****************************************************************/
  printf("\n=== Setting up Debug Proxy ===\n");

  // Create DebugProxyClient
  DebugProxyHandle *debug_proxy = NULL;
  err = debug_proxy_connect_rsd(adapter, handshake, &debug_proxy);
  if (err != IdeviceSuccess) {
    fprintf(stderr, "Failed to create debug proxy client: %d\n", err);
    rsd_handshake_free(handshake);
    adapter_free(adapter);
    return 1;
  }

  /*****************************************************************
   * Interactive Shell
   *****************************************************************/
  printf("\n=== Starting Interactive Debug Shell ===\n");
  printf("Type GDB debugserver commands or 'quit' to exit\n\n");

  char command[MAX_COMMAND_LENGTH];
  bool running = true;

  while (running) {
    printf("debug> ");
    fflush(stdout);

    if (fgets(command, sizeof(command), stdin) == NULL) {
      break;
    }

    // Remove newline
    command[strcspn(command, "\n")] = '\0';

    if (strcmp(command, "quit") == 0) {
      running = false;
      continue;
    }

    // Split command into name and arguments
    char *name = strtok(command, " ");
    char *args = strtok(NULL, "");

    // Create command
    DebugserverCommandHandle *cmd = NULL;
    if (args != NULL && args[0] != '\0') {
      // Split arguments
      char *argv[16] = {0};
      int argc = 0;
      char *token = strtok(args, " ");
      while (token != NULL && argc < 15) {
        argv[argc++] = token;
        token = strtok(NULL, " ");
      }

      cmd = debugserver_command_new(name, (const char **)argv, argc);
    } else {
      cmd = debugserver_command_new(name, NULL, 0);
    }

    if (cmd == NULL) {
      fprintf(stderr, "Failed to create command\n");
      continue;
    }

    // Send command
    char *response = NULL;
    err = debug_proxy_send_command(debug_proxy, cmd, &response);
    debugserver_command_free(cmd);

    if (err != IdeviceSuccess) {
      fprintf(stderr, "Command failed with error: %d\n", err);
      continue;
    }

    if (response != NULL) {
      printf("%s\n", response);
      idevice_string_free(response);
    } else {
      printf("(no response)\n");
    }

    // Read any additional responses
    while (true) {
      err = debug_proxy_read_response(debug_proxy, &response);
      if (err != IdeviceSuccess || response == NULL) {
        break;
      }
      printf("%s\n", response);
      idevice_string_free(response);
    }
  }

  /*****************************************************************
   * Cleanup
   *****************************************************************/
  debug_proxy_free(debug_proxy);
  rsd_handshake_free(handshake);
  adapter_free(adapter);

  printf("\nDebug session ended\n");
  return 0;
}
