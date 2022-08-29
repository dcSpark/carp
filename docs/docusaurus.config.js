// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/dracula');

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Carp Documentation',
  tagline: 'Cardano Postgres Indexer',
  url: 'https://dcspark.github.io',
  baseUrl: '/carp/',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',
  favicon: 'img/favicon.ico',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'dcSpark', // Usually your GitHub org/user name.
  projectName: 'carp', // Usually your repo name.

  // Even if you don't use internalization, you can use this field to set useful
  // metadata like html lang. For example, if your site is Chinese, you may want
  // to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl:
            'https://github.com/dcSpark/carp/',
        },
        theme: {
          customCss: [require.resolve('./src/css/custom.css')],
        },
      }),
    ],
    [
      'redocusaurus',
      {
        // Plugin Options for loading OpenAPI files
        specs: [
          {
            spec: 'bin/openapi.json',
            route: '/openapi/',
          },
        ],
        // Theme Options for modifying how redoc renders them
        theme: {
          // Change with your site colors
          primaryColor: '#052F42',
        },
      },
    ]
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      navbar: {
        title: 'Carp Documentation',
        logo: {
          alt: 'Carp Logo',
          src: 'img/logo.svg',
        },
        items: [
          {
            type: 'doc',
            docId: 'intro',
            position: 'left',
            label: 'Documentation',
          },
          {to: '/openapi', label: 'OpenAPI / Swagger', position: 'left'},
          {
            href: 'https://github.com/dcSpark/carp/',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Docs',
            items: [
              {
                label: 'Tutorial',
                to: '/docs/intro',
              },
            ],
          },
          {
            title: 'Assets',
            items: [
              {
                html: `
                  <a class="footer__link-item" href="/carp/CARP-assets.zip" target="_blank" rel="noreferrer noopener">Brand assets</a>
                `,
              },
            ],
          },
          {
            title: 'Community',
            items: [
              {
                label: 'Discord',
                href: 'https://discord.gg/dcspark',
              },
              {
                label: 'Twitter',
                href: 'https://twitter.com/dcspark_io',
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} dcSpark`,
      },
      colorMode: {
        respectPrefersColorScheme: true,
      },
      prism: {
        additionalLanguages: ['rust', 'toml'],
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
      },
    }),
};

module.exports = config;
