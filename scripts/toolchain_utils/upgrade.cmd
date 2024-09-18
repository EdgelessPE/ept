@echo off
title ept upgrade

echo Waiting for parent process evacuation...
@ping 127.0.0.1 -n 2 >nul

copy /y release\* "{target}"
echo Updated successfully
@ping 127.0.0.1 -n 3 >nul