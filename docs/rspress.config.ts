import * as path from 'node:path';
import { defineConfig } from '@rspress/core';
import { pluginRss } from '@rspress/plugin-rss';
import { pluginSitemap } from '@rspress/plugin-sitemap';
import rspressPluginFileTree from 'rspress-plugin-file-tree';
import rspressPluginMermaid from 'rspress-plugin-mermaid';
import rspressPluginReadingTime from 'rspress-plugin-reading-time';

const siteUrl = 'https://ouywm.github.io/rfeign/';

export default defineConfig({
  base: '/rfeign/',
  root: path.join(__dirname, 'docs'),
  lang: 'zh',
  locales: [
    {
      lang: 'zh',
      label: '简体中文',
      title: 'rfeign',
      description: 'Rust 声明式 HTTP 客户端，灵感来自 OpenFeign / Retrofit。',
    },
    {
      lang: 'en',
      label: 'English',
      title: 'rfeign',
      description:
        'A declarative HTTP client for Rust, inspired by OpenFeign / Retrofit.',
    },
  ],
  ssg: true,
  title: 'rfeign',
  description: 'Rust 声明式 HTTP 客户端',
  plugins: [
    pluginSitemap({
      siteUrl,
      defaultChangeFreq: 'weekly',
    }),
    pluginRss({
      siteUrl,
      output: { dir: 'rss', type: 'rss' },
      feed: {
        id: 'updates',
        title: 'rfeign Docs Updates',
        description: 'rfeign documentation updates',
        language: 'zh-CN',
        copyright: 'Copyright © 2026 rfeign',
      },
    }),
    rspressPluginReadingTime({ defaultLocale: 'zh-CN' }),
    rspressPluginMermaid({ mermaidConfig: { theme: 'neutral' } }),
    rspressPluginFileTree({ initialExpandDepth: 2 }),
  ],
  themeConfig: {
    socialLinks: [
      {
        icon: 'github',
        mode: 'link',
        content: 'https://github.com/ouywm/rfeign',
      },
    ],
  },
});
