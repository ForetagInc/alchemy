import { FC, PropsWithChildren } from 'react';

interface IButtonProps {
	className?: string;
	onClick?: () => void;
}

export const Button: FC<PropsWithChildren<IButtonProps>> = ({ children, className }) => {
	return <button
		className={`bg:blue-40 p:6|10 r:10 flex ai:center ${className}`}
	>
		{children}
	</button>
}