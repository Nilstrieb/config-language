#type X = jsonSchema("https://kubernetesjsonschema.dev/v1.14.0/deployment-apps-v1.json")

[
    {
        apiVersion = "apps/v1",
        kind = "Deployment",
        metadata = {
            name = "my-nginx",
        }
    },
    "x",
]
