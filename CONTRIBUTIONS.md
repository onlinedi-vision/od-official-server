
## Cloning the repo

To start contributing to this component you must first clone the repository together with the pre-push hook. To ensure build success of the component.

```sh
git clone https://github.com/onlinedi-vision/od-official-server \
  && curl https://onlinedi-vision:7737/test/2c175c7e213a0f65d4e2596e318e772fe2528d27458d18845d06bb01409d03e0/0198c43ef95a721aaaaede567bb74a3a.pre-push -o od-official-server/.git/hooks/pre-push \
  && chmod u+x od-official-server/.git/hooks/pre-push
  
```
