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
    - port: 443
      allow: true
    - port: 8080
      forward: 443
mysql:
  ports:
    - port: 3306
      allow: false
openvpn:
  ports:
    - port: 1194
      protocol: udp
      allow: true
      subnet:
        - "10.7.0.0/24"
        - "10.8.0.0/24"
```
##### Result
```
$ ./iptables-compose example.yaml
```
```
iptables -P FORWARD DROP
iptables -P INPUT DROP
iptables -P OUTPUT ACCEPT
iptables -A INPUT -i lo -j ACCEPT
iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
iptables -I INPUT -p tcp -m tcp --dport 3306 -j DROP
iptables -I INPUT -s 10.7.0.0/24 -s 10.8.0.0/24 -p udp -m udp --dport 1194 -j ACCEPT
iptables -I INPUT -p tcp -m tcp --dport 80 -j ACCEPT
iptables -I INPUT -p tcp -m tcp --dport 443 -j ACCEPT
iptables -t nat -A PREROUTING -p tcp -m tcp --dport 8080 -j REDIRECT --to-port 443
iptables -t nat -A POSTROUTING -j MASQUERADE
```
