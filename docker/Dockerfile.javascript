FROM node:16

WORKDIR /app
RUN apt-get update && apt-get install -y curl
EXPOSE 5000
CMD ["tail", "-f", "/dev/null"]