/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',
  webpack: config => {
    config.experiments.asyncWebAssembly = true;
    return config;
  },
};

module.exports = nextConfig;
