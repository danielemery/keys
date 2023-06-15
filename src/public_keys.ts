export interface PublicKey {
  name: string;
  user: string;
  tags: string[];
  key: string;
}

const keys: PublicKey[] = [
  {
    name: "covid-reborn-windows",
    user: "demery",
    tags: [
      // Machine Targets
      "acorn",
      "outpost",
      "thunderbird",
      // Git Targets
      "github-personal",
      "bitbucket-personal",
    ],
    key:
      "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC3o6dpyLFuyDfqhc84es4R2xNE+AhsKKqKJNxs6eyLcqIf9dezH8BD9Ye6E0BoupeZwJx9CL3wwZFmdpHEYmdLb1e7PRxx0hf/6nLRBI5+34gKukj3dZtAhZuiGOQ3sKl6iOqCTi499cRBi2TxdH2xS9n0sZCIWFLuvVzyYy+AX9F1hSTCkVhTvQKc3PJCUZHluk83ydvCyQh0wzUYDVSLkNkt03Ptu2tkj8VqTMsc8WPwBsnBwgNqK3FrD45HuFJYSObEO7ZqrHMZXOyys/jgjoAnIJ+CB5ef43PopTe+IQwqilf8JOjl7PWLPXDpnemiBkPKPy6MBGUr0F9mVEaD",
  },
  {
    name: "de-abusix-keyring",
    user: "demery",
    tags: [
      // Machine Targets
      "acorn",
      "outpost",
      "thunderbird",
      // Git Targets
      "abusix",
      "bitbucket-personal",
      "github-personal",
    ],
    key:
      "sk-ssh-ed25519@openssh.com AAAAGnNrLXNzaC1lZDI1NTE5QG9wZW5zc2guY29tAAAAIJ403siz7L3uzTCFwQ7vM5MMnEnOrFhE1XiMxT0hnmDOAAAABHNzaDo=",
  },
  {
    name: "de-abusix-home",
    user: "demery",
    tags: [
      // Machine Targets
      "acorn",
      "outpost",
      "thunderbird",
      // Git Targets
      "abusix",
      "github-personal",
      "bitbucket-personal",
    ],
    key:
      "sk-ssh-ed25519@openssh.com AAAAGnNrLXNzaC1lZDI1NTE5QG9wZW5zc2guY29tAAAAID+ydUByZyo/wUjG1mGpvxWsg6qD1atoSc2wgjkaQlnKAAAABHNzaDo=",
  },
  {
    name: "de-acorn",
    user: "demery",
    tags: [
      // Git Targets
      "github-personal",
    ],
    key:
      "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQD4RmySuwXCGyCJVidnov3wHY02/Xlzuf3p8QjuRkPgSUbaYugN0kV16Df2P/F+10+oHAfeJTKs+9xHz0cescSkB1Zfn3TxkfsbzX8al84o7e39ijs+TzROM6VtIo+JLyeUnBIYuBrVmyuZEBbHKM5Q1x/dcpByS8wh0NBjdfFi4aVm9q0xtBb7CmxB3WPVN0ZQ4WeiEptx6XMsXv9BJkL8GEU4l6FOU+WJIah+h4pFRKYcL1JVmFCUVWDUdFPpCcCOmEoFh54ci6giPYqF+GA3oJKzoTscLIfmxcYaOvTWZ4v54L52RTbVF23O9yeljO0+iYm8bynt7dQ3b7rq1Bks6ig6kFDs7o74i1Tkrf6TWt761GqM/etxvqeU2y3Qes5Bajl+8nFIa9qvtMzyZqMa4c1aZfjUGTO93QkS+X3WxrqX8VM8RlPyubS6HlvJZmIofiE2zcq8JUV8gKXLZXgGXFfZARwE5pSEn/F1jYN/Nnar645kNGI5zbojcREfZWU=",
  },
  {
    name: "homix-home",
    user: "demery",
    tags: [
      // Machine Targets
      "acorn",
      "outpost",
      "thunderbird",
      // Git Targets
      "github-personal",
      "bitbucket-personal",
    ],
    key:
      "sk-ssh-ed25519@openssh.com AAAAGnNrLXNzaC1lZDI1NTE5QG9wZW5zc2guY29tAAAAIJ3Nvlk87FLF3r5hGeDEX8qX0zwtBgvvoHLEPVnBU8edAAAABHNzaDo=",
  },
  {
    name: "homix-keyring",
    user: "demery",
    tags: [
      // Machine Targets
      "acorn",
      "outpost",
      "thunderbird",
      // Git Targets
      "github-personal",
      "bitbucket-personal",
    ],
    key:
      "sk-ssh-ed25519@openssh.com AAAAGnNrLXNzaC1lZDI1NTE5QG9wZW5zc2guY29tAAAAICAjJChUgcGVHfQt5BgdBx/fpNcqtcy3WmaRCYSjC3GpAAAABHNzaDo=",
  },
  {
    name: "homix-typea",
    user: "demery",
    tags: [
      // Machine Targets
      "acorn",
      "outpost",
      "thunderbird",
      // Git Targets
      "github-personal",
      "bitbucket-personal",
    ],
    key:
      "sk-ssh-ed25519@openssh.com AAAAGnNrLXNzaC1lZDI1NTE5QG9wZW5zc2guY29tAAAAIMqRpAXHVeIF20k3ugchuJavIWoJQh4ar92fccU17GPkAAAABHNzaDo=",
  },
  {
    name: "pgp-yubikey",
    user: "demery",
    tags: [
      // Machine Targets
      "acorn",
      "outpost",
      "thunderbird",
      // Git Targets
      "abusix",
      "github-personal",
      "bitbucket-personal",
    ],
    key:
      "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQC6Ezt4eX71ZxvP7shtcniLJI6N0CWFrc3GSHtq3e/HF3LLWqnY5MWmhREuZrBLPhQX75n6uBWmSsbarwWeHzxO6UJCDkv9s/jMPFv3aV1d49qdF2LvEeKAy8lvN6jTINCEyE3g26tcDPavtciDszILdt+/mDrCWDU/+qKxb3I5X4MS+kj342fqPISJ1J4cfNMskyJib2LliWZnMXfAWgsIVM62Jx5WfJnsyXdjKdahUZeeN+2mr9OFr0ElKY1S3pbCds8BSibsarr9MIfWR4e/0DLWWfmpcPmuOX9lB+3g/bFFmcuyUoVhTMxW4tAG+xSOI89GVWBHx27z5MbGxmRfT2xSXb9DG9EA+p0bx4EUeyc6UIYmKk5R+rfVlagNocLqJDTEWDIum/Xq4qyL5mHXvk4gtVV8AKemqjsSGAtZnBEiAenA5vkVohHLWvq0WtkS9MvghOcG2VGAeuyr4muEGqm0BwrYxqXZBjugmhCcF5rFLSGv+h1s09JO0N5Q8eBmQTBrpif2b2ULnwm5xgVimce4VAgAN3QD0JTDqm2o43m7iS4YsoIXgVh/N5rpTWw0y/60eusxlyUC30mat1oSkYWvzXExIIvINpEGAyVieMvnCtSiSID9qQTFEja18G0Z6XN5eY6/u+ES6krZwWPbDs23auuvE+0nZth8Os9hkw==",
  },
];

export default keys;
