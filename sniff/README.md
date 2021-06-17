# sniff

This is a helper program that can be used to generate traffic between the official Game SDK to Discord, so that it can be sniffed and replicated in the crate. This is done by relaying the traffic in an intermediate Unix doman socket owned by `socat` that will print out all of the traffic that receives.

This currently will only work on Linux, but could be made to work on Mac if someone wants to do that.

## Usage

1. Open Discord, DiscordCanary, or DiscordPTB and login.
1. Run `./sniff/sniff.sh`. If you want to sniff a second discord, add the 0-based index as an argument, eg `./sniff/sniff.sh 1`, this will create a relay unix domain socket owned by socat that will print out the the traffic between the SDK and the Discord application.
1. Run `./run-sniff.sh`, this will compile and run the actual program that uses the SDK, inspect the source to see what commands/arguments are available.
