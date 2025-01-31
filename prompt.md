Let's create a Rust application called Reporteer

The application will interact with the TEE platform inside a confidential container, running in a Kata container in Kubernetes on AMD SEV SNP and Intel TDX.

The application will run inside a Docker container in the configuration above.

The app will call an endpoint configured via env variables, that by default is 127.0.0.1:8006/derived_key

Then it will hash the result gathered from the endpoint above, hash it and keep it in memory.

The hash needs to be readable in the following ways:
* output in the container logs once, at startup
* reached via a nice HTML page that focuses on the derived_key hash
* an api endpoint that exposes it as json

Please give me the content of all the files and I will copy it manually.
Also remember to create tests for everything, follow security patterns and 12 factors.

#lsp
#buffer
#viewport

@rag
