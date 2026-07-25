# Demo 7: Potentiometer (Dicovery Kit KR-0004)

This demo shows how to use a potentiometer through the Micro:bit ADC. Plus, the scrolling effect when displaying the measured value on the 5x5 LED matrix.

**Note:** For this demo, I chose not to use the Micro:bit BSP crate for it was not aligned with the latest version of the other crates I use (like Embassy ones). So I chose to reimplement the scrolling effect without the BSP.