import { NextResponse } from 'next/server';

export async function GET() {
	try {
		const apiKey = process.env.NASA_API_KEY;
		const startDate = new Date().toISOString().split('T')[0];
		const endDate = startDate;

		const res = await fetch(
			`https://api.nasa.gov/neo/rest/v1/feed?start_date=${startDate}&end_date=${endDate}&api_key=${apiKey}`,
			{ cache: 'no-store' } // Disable caching for real-time data
		);

		if (!res.ok) {
			return NextResponse.json(
				{ error: `NASA API returned ${res.status}: ${res.statusText}` },
				{ status: res.status }
			);
		}

		const data = await res.json();
		const nearEarthObjects = data.near_earth_objects?.[startDate] || [];

		return NextResponse.json({ nearEarthObjects });
	} catch (error) {
		console.error('Error fetching NASA NEO data:', error);
		return NextResponse.json(
			{ error: 'Failed to fetch asteroid data' },
			{ status: 500 }
		);
	}
}
