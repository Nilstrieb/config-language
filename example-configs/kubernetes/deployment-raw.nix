[
  {
    apiVersion = "apps/v1";
    kind = "Deployment";
    metadata.name = "my-nginx";
    spec = {
      selector.matchLabels.run = "my-nginx";
      replicas = 2;
      template = {
        metadata.labels.run = "my-nginx";
        spec.containters = [
          {
            name = "my-nginx";
            image = "nginx";
            ports = [{ containerPort = 80; }];
          }
        ];
      };
    };
  }
  {
    apiVersion = "apps/v1";
    kind = "Service";
    metadata = {
      name = "my-nginx";
      labels.run = "my-nginx";
    };
    spec = {
      ports = { port = 80; protocol = "TCP"; };
      selector = { run = "my-nginx"; };
    };
  }
]
