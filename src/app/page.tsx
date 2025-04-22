'use client';

import EnhancedStarryBackground from '@/components/EnhancedStarryBackground';
import ImpactTable from '@/components/ImpactTable';
import { useEffect, useState } from 'react';

export default function Home() {
  const [nearEarthObjects, setNearEarthObjects] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    async function fetchData() {
      try {
        setLoading(true);
        setError(false);

        const response = await fetch('/api/neo-data');

        if (!response.ok) {
          throw new Error(`API returned ${response.status}: ${response.statusText}`);
        }

        const data = await response.json();

        if (!data || !data.nearEarthObjects || data.nearEarthObjects.length === 0) {
          console.error("Invalid data format from API", data);
          setError(true);
          return;
        }

        setNearEarthObjects(data.nearEarthObjects);
      } catch (error) {
        console.error("Error fetching NASA data:", error);
        setError(true);
      } finally {
        setLoading(false);
      }
    }

    fetchData();
  }, []);

  //console.log(nearEarthObjects);

  // Prepare data for ImpactTable
  const headers = ["Name", "Diameter (m)", "Relative Velocity", "Potentially Hazardous"];
  const tableRows = !loading && !error && nearEarthObjects.length > 0
    ? nearEarthObjects.map(obj => [
      obj.name,
      `${Math.round(obj.estimated_diameter?.meters?.estimated_diameter_min || 0)} - ${Math.round(obj.estimated_diameter?.meters?.estimated_diameter_max || 0)}`,
      obj.close_approach_data?.[0]?.relative_velocity.kilometers_per_hour || "no data",
      obj.is_potentially_hazardous_asteroid ? "YES" : "No"
    ])
    : [];

  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <EnhancedStarryBackground />
      <div className="z-10 text-white typewriter-font">
        <h1 className="text-4xl font-bold mb-8 typewriter-font">Asteroid Tracker</h1>
        <p className="text-xl typewriter-font">Welcome to the universe!</p>
        <p className="text-sm mt-4 typewriter-font">Near-earth objects are below:</p>

        {loading && (
          <p className="mt-6 text-center typewriter-font">Loading asteroid data...</p>
        )}

        {error && (
          <p className="mt-6 text-center text-red-500 typewriter-font">
            Error loading asteroid data. Please try again later.
          </p>
        )}

        {!loading && !error && (
          <div className="mt-6">
            <ImpactTable headers={headers} rows={tableRows} />
          </div>
        )}
      </div>

      <style jsx global>{`
        .typewriter-font {
          font-family: 'Special Elite', monospace;
          letter-spacing: 1px;
        }
      `}</style>
    </main>
  );
}
