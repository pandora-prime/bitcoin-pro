# Build and run with Docker

## Requirements

You need to install [Docker](https://docs.docker.com/desktop/).

You will also need [x11docker](https://github.com/mviereck/x11docker), a tool that allows to run graphical applications inside a docker container.  
```
git clone https://github.com/mviereck/x11docker.git
cd x11docker
sudo ./x11docker --install
```

## TL;DR

`make start` 

## Todo
* make the window resizable
* share a volume with host to save the keys
