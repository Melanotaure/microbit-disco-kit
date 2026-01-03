# LCD1602 TEMP

This demo is based on the rusty_lcd1602 demo but enriched with the micro:bit temperature display on the LCD device.

## Goal of the demo

Using **Embassy**, the goal is to show how 2 tasks can run concurrently and by passing an argument to one another.

In this case the "display" task refreshes every 500 ms while the "temperature" task refreshes every 2 s.\
The task "temperature" passes its temperature value to the "display" task using a "Signal" which is like a *RTOS queue* but with one slot.

## Use of Teleplot

Teleplot is an extension of VSCode that plots data directly within VSCode. In order to do so, it reads the port COM and plots what you send as long as it respects the following format: `>name:data`.

In my case, I send the temperature with the format: `>Temp:21` for instance.

But to do so, I need to send the data through the UART. Thanks to **embassy-nrf**, the configuration and the creation of a UART task managing the sending is easy.