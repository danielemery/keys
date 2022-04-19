export interface PublicKey {
  name: string;
  tags: string[];
  key: string;
}

const keys: PublicKey[] = [
  {
    name: "covid-reborn-wsl",
    tags: ["demery", "github-personal", "thunderbird", "bitbucket-personal"],
    key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC14crNaQcR3mONmf29EXOVdOILdWN+H00n2LjwpmNGYJV/70M8+SyM3BFaV8Txj6YN6Qbuyetc9P+8TNU5yzieLmWsT4gwd4Tf1sCz+xbuXm+3MqrZbJKT+5xEgj4/cOngzmKTpiLy7QGqD0I9u/1wdu7AdmEsKV9Pjc479jg37/BRg+JmUHbtAa8cAxQr1/5P+mU+xCoCshseD3oqOhCKwsXToiQGx+hss5r1SRVcND5zCuPFRuNm9NhXAVMSwGfS2GBF0JbWmwlVHfrlIwAvGVzYDJ5HE5/0Zdd+05E1r/UxEV/ZWLlbpTnGgMAueSs3ybF/wAQKhyxSyMTFoTMmsi0pr5ad+z8un4C4GbOI6hkHV0cuvsf/A1XBgLBC/6N88DHA5ICmq7deKU8dlQ6r5eK8gPt0xWmr6RvZ1x7iMrWY9E6BWQXY+97iqf44rmpkMWKP0S+ZuiN95GUBU/+LHPQ6o44c+VeOic/WqoJ1zUWYEfz733+SzYFSgrno0g8=",
  },
  {
    name: "covid-reborn-windows",
    tags: ["demery", "github-personal", "thunderbird", "bitbucket-personal"],
    key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC3o6dpyLFuyDfqhc84es4R2xNE+AhsKKqKJNxs6eyLcqIf9dezH8BD9Ye6E0BoupeZwJx9CL3wwZFmdpHEYmdLb1e7PRxx0hf/6nLRBI5+34gKukj3dZtAhZuiGOQ3sKl6iOqCTi499cRBi2TxdH2xS9n0sZCIWFLuvVzyYy+AX9F1hSTCkVhTvQKc3PJCUZHluk83ydvCyQh0wzUYDVSLkNkt03Ptu2tkj8VqTMsc8WPwBsnBwgNqK3FrD45HuFJYSObEO7ZqrHMZXOyys/jgjoAnIJ+CB5ef43PopTe+IQwqilf8JOjl7PWLPXDpnemiBkPKPy6MBGUr0F9mVEaD",
  },
  {
    name: "junior-wsl",
    tags: ["demery", "github-personal", "thunderbird", "bitbucket-personal"],
    key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDY4t8wCUSyPpfvBZKE/LqDQjP0ZlLXNQqhTPtrU/1lIskxHzslHHus2UOvltyUGcRiER5dSE8WBuCfxdCKPm53K2P9u836ejDqqnUrssIN7Ze3ZQQ6OqO23ESTpsGk+dGig6sG3iZlCkb+Lin/54iQqFeN/gfYpjbs/2V8bsq5jU0eHNekq8hk4OG4NKxZSMPrj6PpNUxMUNlCm/vX1Bis7HGu45EIl726uHwGeD0F28Ckf2NiB79FsU4Jkr9CqORs1INa3lASQDnXnXicaXLdSXTACCKHAGRs98Uzo4vp1qyOy7lDb3pcAOR6Qw9QvZt+5ymf/Lf9DYcWzpWZrPnJGtPQAy3YNpYmderbKpE59dcI9P3PQMSTInsknwEm/C7Jften/O7xSp5o7z/GmD+MyP65G/aafr88wL6SjvacRe24ITCRFeGMOJemSjAdzufCBe9KxE39mlP5kyHVNLRLHbLJNo5md2LvDsxibHbO1O5PfY9Qs1ws6l9bR8auDrM=",
  },
  {
    name: "junior-windows",
    tags: ["demery", "github-personal", "thunderbird", "bitbucket-personal"],
    key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC70NKOOwloddsQ1foaSHxnzTjWK481+creWj6rjedBWU4j/d3GEIsfW2vMMnBfZCRca5yqx5oZv6ujyx2iP3EBAlzORVZVBA2DTy15cD8YhL5ntLcW1CV3UxgwMoDyOklGyIzjeqj4nEqUSayWCEo7BiHQ/rLrkpYI9k+LBRiFSQ3csYBUsaIlECRj/9a57u99hKjoc8qUDsf08oA9gQmAO2uUEBEDUDg5WrqLJuUWi9Yll1X0Wl5uLFrm/KYPGmaRG04c1gJfLwc5ZGlN44LrKv7tU6ndetSOMVStfx7mCMg6yt+cD/7L/pYkGfPWQZNK5RqM6pFQaecrNY/qdyQOsKgM1EHA/KDZ96uDSRASGgyAS8szw11lccemzC/KiP/R+FOMoXaNwYvVXw5cijaUNPgSDIZJGITt1afdwMuEwpKj9pCvxXOH6y4Qj2DKLVCy2gNcXE3g+JMKHVa8m3ahcTC5r3j1X+SnVezZ4Y1Dym9sUk0WMpGL6dEssi1LTMc=",
  },
  {
    name: "tornado-dev-keyring",
    tags: ["demery", "github-personal", "thunderbird", "bitbucket-personal"],
    key: "sk-ssh-ed25519@openssh.com AAAAGnNrLXNzaC1lZDI1NTE5QG9wZW5zc2guY29tAAAAIAvfL0s9vsXItHcjOdOzC95KH4voq79C1654wINWS+7wAAAABHNzaDo=",
  },
  {
    name: "tornado-dev-home",
    tags: ["demery", "github-personal", "thunderbird", "bitbucket-personal"],
    key: "sk-ssh-ed25519@openssh.com AAAAGnNrLXNzaC1lZDI1NTE5QG9wZW5zc2guY29tAAAAIOmv3sL4n9tgrFUdK4nYnmFqzG1MzM5sneo+u8RbFVLsAAAABHNzaDo=",
  },
  {
    name: "de-abusix-keyring",
    tags: ["demery", "abusix", "github-personal", "thunderbird", "bitbucket-personal"],
    key: "sk-ssh-ed25519@openssh.com AAAAGnNrLXNzaC1lZDI1NTE5QG9wZW5zc2guY29tAAAAIJ403siz7L3uzTCFwQ7vM5MMnEnOrFhE1XiMxT0hnmDOAAAABHNzaDo=",
  },
  {
    name: "de-abusix-home",
    tags: ["demery", "abusix", "github-personal", "thunderbird", "bitbucket-personal"],
    key: "sk-ssh-ed25519@openssh.com AAAAGnNrLXNzaC1lZDI1NTE5QG9wZW5zc2guY29tAAAAID+ydUByZyo/wUjG1mGpvxWsg6qD1atoSc2wgjkaQlnKAAAABHNzaDo=",
  },
];

export default keys;
