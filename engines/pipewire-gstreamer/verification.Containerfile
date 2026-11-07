ARG THE_BASE=scratch
FROM ${THE_BASE}
RUN dnf install --assumeyes --setopt=install_weak_deps=False \
      gcc patch pkgconf-pkg-config gstreamer1-devel gstreamer1-plugins-base-devel \
      gstreamer1-plugins-good pipewire-devel pipewire-utils pipewire-pulseaudio \
      pipewire-gstreamer wireplumber pulseaudio-utils dbus-daemon python3 \
 && dnf clean all
COPY engines/pipewire-gstreamer /engine
RUN bash /engine/build.sh /usr/lib/alo-capture-provider/gstreamer-1.0 \
 && bash /engine/test-buffer-wait.sh /usr/lib/alo-capture-provider/gstreamer-1.0 \
 && GST_PLUGIN_PATH_1_0=/usr/lib/alo-capture-provider/gstreamer-1.0 \
      GST_REGISTRY=/tmp/build-registry.bin gst-inspect-1.0 pipewiresink > /tmp/plugin.txt \
 && grep -F '/usr/lib/alo-capture-provider/gstreamer-1.0/libgstpipewire.so' /tmp/plugin.txt \
 && grep -F '1.4.90+ee2c6eb4.2770143f.wait1' /tmp/plugin.txt
WORKDIR /source
CMD ["bash", "engines/pipewire-gstreamer/verify.sh"]
