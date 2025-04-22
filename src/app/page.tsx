'use client';

import React from 'react';
import EnhancedStarryBackground from '@/components/EnhancedStarryBackground';
import ImpactTable from '@/components/ImpactTable';

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <EnhancedStarryBackground />
      <style jsx global>{`
        .typewriter-font {
          font-family: 'Special Elite', monospace;
          letter-spacing: 1px;
        }
      `}</style>
      <div className="z-10 text-white typewriter-font">
        <h1 className="text-4xl font-bold mb-8 typewriter-font">Asteroid Tracker</h1>
        <p className="text-xl typewriter-font">Welcome to the universe!</p>
        <p className="text-sm mt-4 typewriter-font">The nearest objects to Earth are below:</p>
        <ImpactTable
          headers={['Object', 'Distance (km)', 'Velocity (km/s)']}
          rows={[
            ['2024 BX32', '1,234,567', '25.6'],
            ['2025 AB12', '987,654', '18.2'],
            ['2025 QX11', '1,111,222', '20.4']
          ]}
        />

      </div>
    </main>
  );
}
