FROM node:lts-alpine

WORKDIR /app

# Install dependencies
COPY package.json package-lock.json* ./
RUN npm ci --production

# Copy application code
COPY src ./src

# Set environment variables
ENV NODE_ENV=production
ENV NATS_URL=nats://lightpub_nats:4222

# Start the service
CMD ["npm", "run", "start"]
