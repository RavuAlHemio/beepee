FROM archlinux:base-devel as build
RUN pacman -Syu --noconfirm
RUN pacman -S --needed --noconfirm cargo git python

COPY ./ /opt/beepee
WORKDIR /opt/beepee
#RUN python3 cicd/version_stamp.py
RUN cargo build --release --all-targets
RUN cargo test --release


FROM archlinux:base
RUN pacman -Syu --noconfirm

RUN groupadd beepee && useradd -rm -d /opt/beepee -g beepee beepee
USER beepee

COPY --from=0 /opt/beepee/target/release/beepee /opt/beepee
RUN ln -s /config/config.toml /opt/beepee/config.toml

HEALTHCHECK --interval=5m --timeout=10s CMD curl http://localhost:8086/ || exit 1

WORKDIR /opt/beepee
CMD ./beepee

