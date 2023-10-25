FROM rust:1.73.0-bookworm

RUN apt update
RUN apt install -y lsb-release wget software-properties-common gnupg
RUN wget https://apt.llvm.org/llvm.sh
RUN chmod +x llvm.sh
RUN ./llvm.sh 15
RUN ln -s /usr/bin/clang-15 /usr/bin/clang
