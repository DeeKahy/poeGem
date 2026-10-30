self:
{ config, lib, pkgs, ... }:
let
  cfg = config.services.poegem;
in
{
  options.services.poegem = {
    enable = lib.mkEnableOption "poeGem, the Divine Font gem calculator";
    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.poegem;
      defaultText = lib.literalExpression "poegem.packages.\${system}.poegem";
    };
    address = lib.mkOption { type = lib.types.str; default = "127.0.0.1"; };
    port = lib.mkOption { type = lib.types.port; default = 8102; };
    contact = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Added to the User-Agent sent to GGG, poe.ninja and RePoE.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.poegem = {
      description = "poeGem";
      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      environment = { POEGEM_DATA_DIR = "/var/lib/poegem"; }
        // lib.optionalAttrs (cfg.contact != null) { POEGEM_CONTACT = cfg.contact; };
      serviceConfig = {
        ExecStart = "${lib.getExe cfg.package} -addr ${cfg.address}:${toString cfg.port}";
        DynamicUser = true;
        StateDirectory = "poegem"; # the cached gem catalogue and its change log
        Restart = "on-failure";
        RestartSec = 5;
        NoNewPrivileges = true;
        PrivateTmp = true;
        PrivateDevices = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        RestrictNamespaces = true;
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        CapabilityBoundingSet = "";
        SystemCallFilter = [ "@system-service" "~@privileged" ];
        RestrictAddressFamilies = [ "AF_INET" "AF_INET6" "AF_UNIX" ];
      };
    };
  };
}
