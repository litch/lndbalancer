# LNDBalancer

This is a port of my ceebalancer plugin to work for LND.

The motivation is:
- I always want people to be "incentivized" to drive my channels toward balanced
- I always want peers to have a good idea of success rate for a given forward without needing to probe me

To this end, the two main operations are:
- Set channel fees as a proportion of the channel capacity.  This is broken into, for example, quintiles so that it's not super noisy or information-leaky
- Set the htlc_max_msat parameter so I tell peers how big a payment *should not* get dropped by my node.  This is also broken into quintiles so it's not super noisy of information-leaky

It runs periodically against the LND node, asking for a channel list, builds an opinion of those parameters, then sets them.

## Usage

### Prerequisites

Before you start, ensure you have the following installed:
- Rust programming language: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
- `cargo`, Rust's package manager, which comes with Rust by default.
- Access to an LND node with the gRPC interface enabled.

### Configuration

1. **Set Up Configuration File**: Start by creating a configuration file based on the provided `config.yaml.example`. Rename `config.yaml.example` to `config.yaml` and update it with your LND node details.

    ```yaml
    application_port: 8080
    dynamic_fees: true
    dynamic_fee_update_frequency: 100
    dynamic_fee_intervals: 5
    dynamic_fee_min: 100
    dynamic_fee_max: 1000

    sources:
      - endpoint: https://your-endpoint:10009
        macaroon: /path/to/your/mn-admin.macaroon
        cert: /path/to/your/nonsense.cert
    ```

Note on the cert path.  If you are using a self-signed cert ("normal" LND access), provide a path to that file.  Otherwise you can give it a nonsense path and it will try to use CA-based certs (for example, if you are running this against a Voltage node).

2. **Ensure LND Accessibility**: Confirm that the LND node is accessible from the machine where LNDBalancer will run, using the `endpoint`, `macaroon`, and `cert` specified in the `config.yaml`.

### Building the Application

1. Clone the LNDBalancer repository (assuming it's available on GitHub or another Git hosting service).

    ```shell
    git clone https://github.com/litch/lndbalancer.git
    cd lndbalancer
    ```

2. Build the project using `cargo`.

    ```shell
    cargo build --release
    ```

    This step compiles the application and produces an executable in `target/release/`.

### Running LNDBalancer

1. Run the LNDBalancer application.

    ```shell
    cargo run --release
    ```

    Alternatively, you can directly execute the binary produced in the `target/release/` directory.

2. **Monitoring**: Once running, LNDBalancer will begin its operations, periodically adjusting channel fees and `htlc_max_msat` parameters based on your LND node's channel states. You can monitor the application's activities through its log output.

### Health Check

- LNDBalancer includes a basic web server for health checks. You can verify that the application is running by accessing `http://localhost:8080/health` (or the port you specified in your `config.yaml`).

### Stopping the Application

- To stop LNDBalancer, simply interrupt the process in your terminal (commonly with `Ctrl+C`).

# License

Run it if you want, copy it if you want, I don't care.

Copyright (c) 2024 Justin Litchfield

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.