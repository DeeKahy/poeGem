
FROM node:20-slim

# Create app directory
WORKDIR /app

# Copy package.json and package-lock.json
COPY package.json ./

# Install dependencies
RUN npm install

# Copy application code
COPY app.js ./
COPY public ./public

# Create cache directory
RUN mkdir -p cache

# Expose the port the app runs on
EXPOSE 3000

# Command to run the application
CMD ["node", "app.js"]