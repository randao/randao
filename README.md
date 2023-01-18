# Findora Randao Implement
Randao implementation for Findora BlockChain.

## Findora Rosetta Docker build and run
### Findora Rosetta image build and run single container
```bash
docker build . -t findora-randao
docker run -p 80:80 -p 8080:8080 -p 9090:9090 -v $(pwd)/campaigns/participant0:/tmp/.randao/campaigns -v $(pwd)/config/config0.json:/tmp/.randao/config/config.json -v $(pwd)/keys:/tmp/.randao/keys -itd --name findora-randao --restart always findora-randao
```
### Findora Rosetta image build and multi run multiple container
```bash
docker-compose up -d
```

