This project contains a solution to Google Code Jam 2008 Round 2
Problem A -- Cheating a Boolean Tree. The problem can be found at:

    https://code.google.com/codejam/contest/32001/dashboard#s=p0

The idea is that you read in a boolean binary tree, where the leafs
are boolean values (i.e., true and false), and the interior nodes
contain the boolean gates/operations AND and OR. Furthermore, some
interior nodes are changeable (i.e., you can toggle them from AND to
OR or vice versa) and others are not..

As specified, you can evaluate the three and determine its computed
boolean value. But you're also given a desired value that may not
match. The problem is to determine the minimum number of toggles
required to achieve the desired boolean value, or determine that it's
impossible to achieve.

The problem lends itself to a very nice recursive solution, as one
might expect given a recursive data structure.

This is my first Rust program. In order to create it I ended up having
to create a scanner library to help me read in the input file, that in
this case simply contained unsigned integers. I simply created the
minimal version to get through this problem, with the idea that it
could be extended later. It's modeled after Java's Scanner class (see:
http://docs.oracle.com/javase/7/docs/api/java/util/Scanner.html).

I went through a number of iterations to create better and better
versions. The current best version is in:

    cheating-a-boolean-tree-3.rs
