import type { NextConfig } from 'next';

const isDevelopment = process.env.NODE_ENV === 'development';

const nextConfig: NextConfig = {
  experimental: {
    ppr: true,
  },
  images: {
    remotePatterns: [
      {
        hostname: 'avatar.vercel.sh',
      },
    ],
  },
  // 開発環境では動的レンダリングを使用し、本番環境では静的エクスポートを使用
  ...(isDevelopment ? {} : { 
    // Tauriでの利用のため静的なHTMLを出力
    output: 'export',
    // Tauriで使用するときはルートからの相対パスではなく、出力ファイルからの相対パスを使用
    basePath: '',
    // 出力ディレクトリを'out'に設定
    distDir: 'out',
  }),
};

export default nextConfig;
