import { defineSiteConfig, readMessages } from './shared/config'

export default defineSiteConfig(
	{
		title: 'Full Steam Ahead',
		url: 'https://full-steam-ahead.creeperkatze.dev',
		repo: 'creeperkatze/full-steam-ahead',
		version: process.env.VERSION,
		messages: readMessages(new URL('../src/locales', import.meta.url)),
		nav: (t) => [
			{
				text: t('nav.translate'),
				link: 'https://crowdin.com/project/full-steam-ahead',
				target: '_blank',
			},
		],
		socialLinks: [{ icon: 'discord', link: 'https://link.creeperkatze.dev/discord' }],
	},
	{
		themeConfig: {
			logo: '/icon.svg',
			siteTitle: false,
		},
	},
)
