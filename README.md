# load-balancer
Proxy-Based Load Balancer with Adaptive Decision Engine

This repo implements both RoundRobin and LeastConnections based algorithm.
One can dymanically choose an algorithm to use to compute the next_worker.
next_worker accepts LoadBalancerAlgorithm(RoundRobin/LeastConnectios) 
as a parameter and executes the corresponding algorithm to compute 
the correct next_worker.

This repo maintains(increment/decrement) the number of active connections 
(used by Least Connections Algorithm) for each worker. 
It also maintains next_worker data (used by Round Robin Algorithm)

Example: there are 5 workers 
         W1 with 2 active connections 
         W2 with 1 active connection
         W3 with 1 active connection
         W4 with 0 active connection 
         W5 with 3 active connections

        next_worker is pointing to W1

        1. invoke next_worker function with RoundRobin as input parameter,
           it will return W1
           it will update active connections to 3 for W1
           next_worker will point to W2

        2. invoke next_worker function with RoundRobin as input parameter,
           it will return W2
           it will update active connections to 2 for W2
           next_worker will point to W3

        3. invoke next_worker function with LeastConnections as input parameter
           it will return W4
           it will update active connection to 1 for W4
           next_worker will point to W5

        Now the status is as follows:

        W1 with 3 active connections
        W2 with 2 active connections
        W3 with 1 active connection
        W4 with 1 active connection
        W5 with 3 active connection

        next_worker is pointing to W5

        4. If we invoke next_worker function with LeastConnections as input parameter
           it will return W3
           it will update active connection to 2 for W3
           next_worker will point to W4

           but if invoke next_worker function with RoundRobin as input parameter
           it will return W5
           it will update active connection to 4 for W5
           next_worker will point to W1

        It also has a function dec function to reduce the number of active connection
        where it takes host address (http://localhost:3000) as input parameter.



         


