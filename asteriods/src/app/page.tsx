'use client';

import React from 'react';
import StarryBackground from '@/components/StarryBackground';

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <StarryBackground />
      <div className="z-10 text-white">
        <h1 className="text-4xl font-bold mb-8">Asteroids</h1>
        <p className="text-xl">An app to monitor Near Earth Objects, powered by NASA's NeoWs API</p>
      </div>
    </main>
  );
}
