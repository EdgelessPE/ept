@echo off
title ept upgrade

echo Info: Waiting for parent process evacuation...
@ping 127.0.0.1 -n 2 >nul

copy /y release\* "{target}"

echo Success: Updated successfully
@ping 127.0.0.1 -n 3 >nul
exit 0