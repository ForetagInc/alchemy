import type { ReactElement } from 'react';
import type { NextPageWithLayout } from '../_app';

import { useState } from 'react';
import { Layout } from '../../layout/Default';

interface ICollection {
	name: string;
}

const ListCollectionPage: NextPageWithLayout = () => {

	const [collections, setCollections] = useState<ICollection[]>([
		{
			name: 'accounts'
		},
		{
			name: 'orders'
		},
	]);

	return (
		<div className='grid grid-cols:8 gap:10'>
			{
				collections.map(({ name }, index) => (
					<div
						key={index}
						className={`bg:gray-16 bg:gray-14:hover r:10 p:10 cursor:pointer min-h:48
							flex ai:center
						`}
					>
						<i className='ri-file-list-3-line mr:16' />
						<p>{name}</p>
					</div>
				))
			}
		</div>
	)
};

ListCollectionPage.getLayout = function getLayout(page: ReactElement) {
	return (
		<Layout>
			{page}
		</Layout>
	)
};

export default ListCollectionPage;