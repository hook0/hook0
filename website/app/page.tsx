import { Testimonials } from "@/components/testimonials";
import { EverythingYouNeed } from "@/components/everything-you-need";
import { Header } from "@/components/header";
import { OutOfTheBox } from "@/components/out-of-the-box";
import { UseCases } from "@/components/use-cases";
import { Pricing } from "@/components/ui/pricing";

export default function Home() {
  return (
    <div>
      <Header />
      <OutOfTheBox />
      <EverythingYouNeed />
      <Testimonials />
      <UseCases />
      <Pricing />
    </div>
  );
}
