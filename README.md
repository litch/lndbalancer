# LNDBalancer

This is a port of my ceebalancer plugin to work for LND.

The motivation is:
- I always want people to be "incentivized" to drive my channels toward balanced
- I always want peers to have a good idea of success rate for a given forward without needing to probe me

To this end, the two main operations are:
- Set channel fees as a proportion of the channel capacity.  This is broken into, for example, quintiles so that it's not super noisy or information-leaky
- Set the htlc_max_msat parameter so I tell peers how big a payment *should not* get dropped by my node.  This is also broken into quintiles so it's not super noisy of information-leaky

It runs periodically against the LND node, asking for a channel list, builds an opinion of those parameters, then sets them.

# Usage

We'll figure it out

# License

Run it if you want, copy it if you want, I don't care.

Copyright (c) 2024 Justin Litchfield

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.