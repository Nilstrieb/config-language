let
  trivialWebService = { name, selectorLabel, port, extraPodLabels ? { } }:
    [
      {
        apiVersion = "apps/v1";
        kind = "Deployment";
        metadata = {
          inherit name;
        };
        spec = {
          selector.matchLabels = selectorLabel;
          replicas = 2;
          template = {
            metadata.labels = selectorLabel // extraPodLabels;
            spec.containters = [
              {
                inherit name;
                image = "nginx";
                ports = [{ containerPort = port; }];
              }
            ];
          };
        };
      }
      {
        apiVersion = "apps/v1";
        kind = "Service";
        metadata = {
          inherit name;
          labels = selectorLabel;
        };
        spec = {
          ports = { port = 80; protocol = "TCP"; };
          selector = selectorLabel;
        };
      }
    ]
  ;
in
trivialWebService {
  name = "my-nginx";
  selectorLabel = { run = "my-nginx"; };
  port = 80;
  extraPodLabels = { someOther = "label"; };
}
