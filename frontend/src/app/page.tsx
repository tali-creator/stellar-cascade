"use client"

import Header from "@/components/landing/Header";
import Hero from "@/components/landing/Hero";
import Marquee from "@/components/landing/Marquee";
import HowItWorks from "@/components/landing/HowItWorks";
import FundingModes from "@/components/landing/FundingModes";
import WhySoroban from "@/components/landing/WhySoroban";
import Cta from "@/components/landing/Cta";
import Footer from "@/components/landing/Footer";

export default function Home() {
  return (
    <main className="bg-[#0A1410]">
      <Header />
      <Hero />
      <Marquee />
      <HowItWorks />
      <FundingModes />
      <WhySoroban />
      <Cta />
      <Footer />
    </main>
  );
}
