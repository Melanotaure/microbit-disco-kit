# HC-SR04 ultrasonic distance sensor

In this example, I added the management of the HC-SR04 sensor to the previous example *lcd1602_temp*.\
I just modified the info displayed on the LCD with the temperature on the first line and the distance on the second line.

I also output the temperature data plus the distance data on the UART in order to feed Teleplot and get a graphic from those data.

This time, I didn't use any crate for the ultrasonic sensor as it is easy to use with Embassy.