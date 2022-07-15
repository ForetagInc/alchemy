import GraphiQL from 'graphiql';
import 'graphiql/graphiql.min.css';

import type { ReactElement } from 'react';
import type { NextPageWithLayout } from './_app';

import { Layout } from '../layout/Default';

const PlaygroundPage: NextPageWithLayout = () => {
	return <GraphiQL
		fetcher={async graphQLParams => {
			const data = await fetch(
				'https://swapi-graphql.netlify.app/.netlify/functions/index',
				{
					method: 'POST',
					headers: {
						Accept: 'application/json',
						'Content-Type': 'application/json',
					},
					body: JSON.stringify(graphQLParams),
					credentials: 'same-origin',
				},
			);
			return data.json().catch(() => data.text());
		}}
	/>
};

PlaygroundPage.getLayout = function getLayout(page: ReactElement) {
	return (
		<Layout>
			{page}
		</Layout>
	)
};

export default PlaygroundPage;