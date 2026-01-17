{ self }:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfgApi = config.services.benben-api-next;
  cfgPreprocessor = config.services.benben-circle-preprocessor;

  sharedSrvCfg = {
    Restart = "always";
    DynamicUser = true;
    ProtectSystem = "strict";
    ProtectHome = true;
    PrivateTmp = true;
    PrivateDevices = true;
    NoNewPrivileges = true;
  };

  commonCfg = {
    databaseUrl = lib.mkOption {
      type = lib.types.str;
      description = "Database connection string (PostgreSQL)";
    };

    envVars = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = { };
      description = "Extra environment variables for the services.";
    };
  };
in
{
  options.services = {
    benben-api-next = commonCfg // {
      enable = lib.mkEnableOption "benben.sbs's backend";

      corsAllowFile = lib.mkOption {
        type = lib.types.nullOr lib.types.path;
        default = null;
        description = "Path to a file containing a line-separated list of allowed CORS origins.";
      };
    };
    benben-circle-preprocessor = commonCfg // {
      enable = lib.mkEnableOption "benben.sbs's circle preprocessor";
    };
  };

  config = {
    systemd.services = {
      benben-api-next = lib.mkIf cfgApi.enable {
        wantedBy = [ "multi-user.target" ];
        after = [ "network.target" ];
        serviceConfig = sharedSrvCfg // {
          ExecStart = "${self.packages.${pkgs.system}.benben-api-next}/bin/benben-api-next";
          StateDirectory = "benben-api-next";
          WorkingDirectory = "/var/lib/benben-api-next";
        } // lib.optionalAttrs (cfgApi.corsAllowFile != null) {
          BindReadOnlyPaths = [ "${cfgApi.corsAllowFile}:/var/lib/benben-api-next/.cors_allow" ];
        };
        environment = {
          DATABASE_URL = cfgApi.databaseUrl;
        } // cfgApi.envVars;
      };

      benben-circle-preprocessor = lib.mkIf cfgPreprocessor.enable {
        wantedBy = [ "multi-user.target" ];
        after = [ "network.target" ];
        serviceConfig = sharedSrvCfg // {
          ExecStart = "${self.packages.${pkgs.system}.circle-preprocessor}/bin/circle-preprocessor";
          StateDirectory = "benben-circle-preprocessor";
          WorkingDirectory = "/var/lib/benben-circle-preprocessor";
        };
        environment = {
          DATABASE_URL = cfgPreprocessor.databaseUrl;
        } // cfgPreprocessor.envVars;
      };
    };
  };
}
