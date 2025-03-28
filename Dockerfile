# Use the official ROS Jazzy desktop image as the base
FROM osrf/ros:jazzy-desktop

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    cmake \
    pkg-config \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust (using rustup)
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Add Cargo and Rust to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Verify installation
RUN rustc --version && cargo --version

# Set ROS environment variables
SHELL ["/bin/bash", "-c"]
RUN echo "source /opt/ros/jazzy/setup.bash" >> ~/.bashrc

# Clone a Rust project from GitHub
ARG REPO_URL="https://github.com/your-username/your-rust-repo.git"
WORKDIR /workspace

# Clone the repository and build it
RUN git clone ${REPO_URL} project && \
    cd project && \
    cargo build --release

# Export the binary
RUN mkdir -p /output && \
    cp /workspace/project/target/release/* /output

# Set the working directory
WORKDIR /workspace/project

# Default command to run when the container starts
CMD ["/bin/bash"]
