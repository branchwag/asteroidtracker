import React from 'react';

interface ImpactTableProps {
	headers: string[];
	rows: string[][];
}

const ImpactTable: React.FC<ImpactTableProps> = ({ headers, rows }) => {
	return (
		<div className="overflow-x-auto w-full max-w-4xl mx-auto mt-8">
			<table className="min-w-full bg-black text-white border border-gray-700 rounded-lg shadow-lg">
				<thead>
					<tr>
						{headers.map((header, idx) => (
							<th key={idx} className="px-4 py-2 border-b border-gray-700 text-left">
								{header}
							</th>
						))}
					</tr>
				</thead>
				<tbody>
					{rows.map((row, rowIdx) => (
						<tr key={rowIdx} className="hover:bg-gray-800">
							{row.map((cell, cellIdx) => (
								<td key={cellIdx} className="px-4 py-2 border-b border-gray-700">
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
