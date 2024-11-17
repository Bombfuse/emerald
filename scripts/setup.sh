# setup act locally
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# setup playdate sdk location
export PLAYDATE_SDK_PATH=$(pwd)/playdate_sdk