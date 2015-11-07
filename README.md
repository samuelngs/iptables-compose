# iptables-compose
YAML files as iptables configuration sources

##### Source
```
filter:
  input: drop
  forward: drop
  output: accept
web:
  ports:
    - port: 80
      allow: true
      subnet:
        - "10.1.0.0/24"
        - "10.2.0.0/24"
    - port: 443
      allow: true
      subnet:
        - "10.1.0.0/24"
        - "10.2.0.0/24"
    - port: 8080
      forward: 443
openvpn:
  ports:
    - port: 1194
      protocol: udp
      allow: true
```
##### Result
```
$ ./iptables-compose example.yaml --reset
```
```
iptables -F
iptables -X
iptables -t nat -F
iptables -t nat -X
iptables -t mangle -F
iptables -t mangle -X
iptables -P FORWARD DROP
iptables -P INPUT DROP
iptables -P OUTPUT ACCEPT
iptables -I INPUT -p udp -m udp --dport 1194 -j ACCEPT
iptables -I INPUT -s 10.1.0.0/24,10.2.0.0/24 -p tcp -m tcp --dport 80 -j ACCEPT
iptables -I INPUT -s 10.1.0.0/24,10.2.0.0/24 -p tcp -m tcp --dport 443 -j ACCEPT
iptables -t nat -A PREROUTING -p tcp -m tcp --dport 8080 -j REDIRECT --to-port 443
```
