export interface PublicKey {
  name: string;
  tags: string[];
  key: string;
}

const keys: PublicKey[] = [
  {
    name: "covid-reborn-wsl",
    tags: ["github", "thunderbird", "bitbucket"],
    key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC14crNaQcR3mONmf29EXOVdOILdWN+H00n2LjwpmNGYJV/70M8+SyM3BFaV8Txj6YN6Qbuyetc9P+8TNU5yzieLmWsT4gwd4Tf1sCz+xbuXm+3MqrZbJKT+5xEgj4/cOngzmKTpiLy7QGqD0I9u/1wdu7AdmEsKV9Pjc479jg37/BRg+JmUHbtAa8cAxQr1/5P+mU+xCoCshseD3oqOhCKwsXToiQGx+hss5r1SRVcND5zCuPFRuNm9NhXAVMSwGfS2GBF0JbWmwlVHfrlIwAvGVzYDJ5HE5/0Zdd+05E1r/UxEV/ZWLlbpTnGgMAueSs3ybF/wAQKhyxSyMTFoTMmsi0pr5ad+z8un4C4GbOI6hkHV0cuvsf/A1XBgLBC/6N88DHA5ICmq7deKU8dlQ6r5eK8gPt0xWmr6RvZ1x7iMrWY9E6BWQXY+97iqf44rmpkMWKP0S+ZuiN95GUBU/+LHPQ6o44c+VeOic/WqoJ1zUWYEfz733+SzYFSgrno0g8=",
  },
];

export default keys;
