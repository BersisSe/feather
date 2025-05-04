cd ..
docker build -f benches.Dockerfile -t %1-bench --build framework=%1 .
docker run --rm -d -p 3000:3000 %1-bench
docker run --rm ghcr.io/william-yeh/wrk -t10 -c400 -d10s http://host.docker.internal:3000/