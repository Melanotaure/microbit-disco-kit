# LCD1602 TEMP

This demo is based on the rusty_lcd1602 demo but enriched with the micro:bit temperature display on the LCD device.

## Goal of the demo

Using **Embassy**, the goal is to show how 2 tasks can run concurrently and by passing an argument to one another.

In this case the "display" task refreshes every 500 ms while the "temperature" task refreshes every 2 s.\
The task "temperature" passes its temperature value to the "display" task using a "Signal" which is like a *RTOS queue* but with one slot.