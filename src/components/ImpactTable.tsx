import React from 'react';

interface ImpactTableProps {
	headers: string[];
	rows: string[][];
}

const ImpactTable: React.FC<ImpactTableProps> = ({ headers, rows }) => {
	return (
		<div className="overflow-x-auto rounded-2xl shadow-lg bg-black p-4">
			<table className="min-w-full text-sm text-left text-white">
				<thead className="text-xs uppercase bg-zinc-900 text-gray-400">
					<tr>
						{headers.map((header, idx) => (
							<th key={idx} className="px-6 py-3">
								{header}
							</th>
						))}
					</tr>
				</thead>
				<tbody>
					{rows.map((row, i) => (
						<tr
							key={i}
							className="border-b border-zinc-800 hover:bg-zinc-800 transition duration-150"
						>
							{row.map((cell, j) => (
								<td key={j} className="px-6 py-4 whitespace-nowrap text-gray-200">
									{cell}
								</td>
							))}
						</tr>
					))}
				</tbody>
			</table>
		</div>
	);
};

export default ImpactTable;
