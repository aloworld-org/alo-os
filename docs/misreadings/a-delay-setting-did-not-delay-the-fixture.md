# A delay setting did not delay the fixture

**Concluded.** Setting `timestamp-offset=750000000` on live `audiotestsrc`
exercised a microphone that starts 750 ms after video. The passing recording
fixture was initially described as testing that boundary.

**What was true.** In this configuration the first generated audio timestamp
was still near zero, with or without `do-timestamp`. A one-time mutation check
removed the new audio-start gate and the fixture still passed. That additional
check caught the false coverage claim before installed acceptance.

**The mechanism.** A property was accepted by the engine, but its effect on the
actual stream was never measured. Successful configuration and successful
decoding established neither the intended delay nor the regression's presence.

**The cure.** The verification-only fixture now delays first delivery and shifts
the generated sample timeline by the observed delay. The repeated PipeWire
fixture checks decoded audio/video onset and refuses WebM timestamp errors.
A one-time mutation measurement without the audio-start gate now fails with
first video at zero and audio at 752.5 ms. The normal gate is restored before
building the installed application; test instrumentation is excluded from it.
