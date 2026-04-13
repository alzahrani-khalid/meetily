/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: false, // Disabled for BlockNote compatibility
  output: 'export',
  images: {
    unoptimized: true,
  },
  // Add basePath configuration
  basePath: '',
  assetPrefix: '/',

  // Add webpack configuration for Tauri
  webpack: (config, { isServer, dev }) => {
    // Use non-inline source maps in dev mode. The default devtool wraps
    // modules in string-based code execution which breaks picomatch and
    // framer-motion (they contain regex patterns with literal /*)
    if (dev) {
      config.devtool = 'cheap-module-source-map';
    }
    if (!isServer) {
      config.resolve.fallback = {
        ...config.resolve.fallback,
        fs: false,
        path: false,
        os: false,
      };
    }
    return config;
  },
}

module.exports = nextConfig
