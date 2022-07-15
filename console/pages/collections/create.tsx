import type { NextPage } from 'next';
import { useState } from 'react';
import { Button } from '../../components';

interface IField {
	name: string;
	required: boolean;
	array?: boolean;
	default?: string;
	type: 'text' | 'number';
}

const CreateCollectionPage: NextPage = () => {

	const [name, setName] = useState('');
	const [fields, setFields] = useState<IField[]>([]);

	return (
		<div className='bg:gray-20 f:white p:8|72'>
			<div className='flex flex:column'>
				<label className='mb:8'>Collection</label>
				<input
					className='outline:none p:8'
					value={name}
					onChange={(e) => setName(e.target.value)}
					placeholder='Collection name e.g. users'
				/>
			</div>

			<div className='mt:20'>
				<div className='flex jc:space-between mb:32'>
					<h2>Properties {name && 'of a ' + name}</h2>
					<button
						className='bg:blue-40 p:6|10 r:12 flex ai:center'
						onClick={() => {
							setFields([...fields, { name: '', type: 'text', required: false }]);
						}}
					>
						<i className='ri-add-box-line mr:4'></i>
						Add field
					</button>
				</div>
				<ul>
					{
						fields.map(({ name, type, required, ...optionals }, index) => (
							<li key={index} className='flex gap:10 jc:space-between mb:10'>
								<input
									required
									placeholder='Field name e.g. firstName'
									value={name}
									onChange={(e) => { setFields(fields.map((field, i) => i === index ? { ...field, name: e.target.value } : field)) }}
								/>

								<select>
									<option>UUID</option>

									<option>Text</option>
									<option>Email</option>
									<option>IPv4</option>
									<option>IPv6</option>
									<option>IPv4/6</option>

									<option>Integer</option>
									<option>Float</option>
									<option>Boolean</option>

									<option>File</option>

									<option>Date</option>
									<option>Time</option>
									<option>DateTime</option>
									<option>DateTime w/ Timezone</option>

									<option>LineString</option>
									<option>Coordinate</option>
									<option>Polygon</option>

									<option>Enum</option>

									<option>Object</option>
								</select>

								<input
									required
									placeholder='Default'
									value={optionals.default}
									onChange={(e) => { setFields(fields.map((field, i) => i === index ? { ...field, default: e.target.value } : field)) }}
								/>

								<label>
									Array
									<input
										type='checkbox'
										checked={optionals.array}
										onChange={(e) => { setFields(fields.map((field, i) => i === index ? { ...field, array: e.target.checked } : field)) }}
										className='ml:10'
									/>
								</label>

								<label>
									Required
									<input
										type='checkbox'
										checked={required}
										onChange={(e) => { setFields(fields.map((field, i) => i === index ? { ...field, required: e.target.checked } : field)) }}
										className='ml:10'
									/>
								</label>

								<i
									className='ri-close-line cursor:pointer'
								/>
							</li>
						))
					}
				</ul>

				<Button className='mt:32'>
					Create
				</Button>
			</div>
		</div>
	)
};

export default CreateCollectionPage;