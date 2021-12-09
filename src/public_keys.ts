export interface PublicKey {
  name: string;
  tags: string[];
  key: string;
}

const keys: PublicKey[] = [
  {
    name: "covid-reborn-wsl",
    tags: ["demery", "github", "thunderbird", "bitbucket-personal"],
    key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC14crNaQcR3mONmf29EXOVdOILdWN+H00n2LjwpmNGYJV/70M8+SyM3BFaV8Txj6YN6Qbuyetc9P+8TNU5yzieLmWsT4gwd4Tf1sCz+xbuXm+3MqrZbJKT+5xEgj4/cOngzmKTpiLy7QGqD0I9u/1wdu7AdmEsKV9Pjc479jg37/BRg+JmUHbtAa8cAxQr1/5P+mU+xCoCshseD3oqOhCKwsXToiQGx+hss5r1SRVcND5zCuPFRuNm9NhXAVMSwGfS2GBF0JbWmwlVHfrlIwAvGVzYDJ5HE5/0Zdd+05E1r/UxEV/ZWLlbpTnGgMAueSs3ybF/wAQKhyxSyMTFoTMmsi0pr5ad+z8un4C4GbOI6hkHV0cuvsf/A1XBgLBC/6N88DHA5ICmq7deKU8dlQ6r5eK8gPt0xWmr6RvZ1x7iMrWY9E6BWQXY+97iqf44rmpkMWKP0S+ZuiN95GUBU/+LHPQ6o44c+VeOic/WqoJ1zUWYEfz733+SzYFSgrno0g8=",
  },
  {
    name: "covid-reborn-windows",
    tags: ["demery", "github", "thunderbird", "bitbucket-personal"],
    key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC3o6dpyLFuyDfqhc84es4R2xNE+AhsKKqKJNxs6eyLcqIf9dezH8BD9Ye6E0BoupeZwJx9CL3wwZFmdpHEYmdLb1e7PRxx0hf/6nLRBI5+34gKukj3dZtAhZuiGOQ3sKl6iOqCTi499cRBi2TxdH2xS9n0sZCIWFLuvVzyYy+AX9F1hSTCkVhTvQKc3PJCUZHluk83ydvCyQh0wzUYDVSLkNkt03Ptu2tkj8VqTMsc8WPwBsnBwgNqK3FrD45HuFJYSObEO7ZqrHMZXOyys/jgjoAnIJ+CB5ef43PopTe+IQwqilf8JOjl7PWLPXDpnemiBkPKPy6MBGUr0F9mVEaD",
  },
  {
    name: "sine-xps-13",
    tags: [
      "demery",
      "disabled",
      "github",
      "thunderbird",
      "bitbucket-sine",
      "bitbucket-personal",
    ],
    key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDWsLeX40/yML9hQGibEsTl08QpL7NeKgnio1SYk13FJUQavgXWcYIvcDiK0kHzANOLDNa63LjiqGpvNLQU6d8wHPyx0a0T83khelGEuikUoLDBW6oOKSlC5aGufdsBMDIxbcR0uImiQo2UE4fUQKajG2ShrJ4QvvILPWTba2CAMZpPy/0TFRhLUqbOtkS2YQNDA4wC80sBeyqzfUhRi7KodCLQ53j+6Rbp7UnM0OofYyqGTJQMsAOVA9yTa6MDa+I1ho4SY+TYrDl5cJbIbJqqQV/vJjCEPpubLvO5DVrQQaJIvUOgegU7IpdzovMcdBYPUIwjj9jXjg+jpNJYL64U8XMTz3oeiKp5b2+KcyWPp4kVruwCmVpXpSBvBy07TGCBSnajk9R8e+K85KipVupO8O6Ax4IovmHtk+MHQwTxgdm6qk53RB29c3b2DMsUFAW/gDvpYL2aMk4VjGn5GL1GMpCYE+tmZiMzdk56KDT4kGVvmgJihOhe//VvJlPBtvs=",
  },
  {
    name: "sine-xps-13-keyring",
    tags: [
      "demery",
      "github",
      "thunderbird",
      "bitbucket-sine",
      "bitbucket-personal",
    ],
    key: "sk-ssh-ed25519@openssh.com AAAAGnNrLXNzaC1lZDI1NTE5QG9wZW5zc2guY29tAAAAIAatA7ahQltP4DtT9sNWmSHlACv743Dh97d21i8AS09RAAAABHNzaDo=",
  },
  {
    name: "sine-xps-13-backup",
    tags: [
      "demery",
      "github",
      "thunderbird",
      "bitbucket-sine",
      "bitbucket-personal",
    ],
    key: "sk-ssh-ed25519@openssh.com AAAAGnNrLXNzaC1lZDI1NTE5QG9wZW5zc2guY29tAAAAIJwFHVpsYsxC0JkW6SnkrEo+NEd360kcHO73mqCiGMr2AAAABHNzaDo=",
  },
];

export default keys;
