# swizzy_decent
A decentralized network
## Background
This project is a continuation of another project written in nodejs called decent.
This project was never released, but was developed to the point where it worked, and acted as the PoC
for this project, and is the source of concepts that will be implemented for this project in rust.

Why rust?

As a friend put it, there is no other reasonable reason to chose a programming language
aside from it's trendiness and meme factor.

But really, because I'm dissapointed that although computers (until we reached the physical limits)
scaled exponentially every 18 months (moors law, or however you spell it), our software run exponentially
slower than what was written at that time. We made a lot of trade offs for speed to market and maintainability
vs performance and resource utilization reduction. Don't get me wrong, I love java and js and any language I've worked with,
and am grateful for them, they lead to exponential growth in productiviy and innovation. I just want to
optimized as much as is in my control, without having to worry so much about the mistakes that are so easy to
make in C. So here we are!


## Project HLD
### HealthCheck
TODO: 




# Cli
Perform health check on remote host:
```bash
hc --remote_addr=127.0.0.1 --port=3450
```

# Network table lookup
Navigate to root http address of host with same port as the health check port.