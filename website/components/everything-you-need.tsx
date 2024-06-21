import { everythingYouNeed } from "@/app/data";

export const EverythingYouNeed = () => {
  {
    /* 
  Make a 3x3 grid card with border, shadows, hover effect, ... with data in ../data/index.ts everythingYouNeed i need that's will be reponsive
  Data format = title: string, decsription: string , icon: IconType, color: string
  IconType is a string that will be used to import the icon from react-icons
*/
  }

  return (
    <section className="mt-36" id="everything-you-need">
      <h2 className="text-4xl font-bold text-center mt-16 mb-10 text-indigo-400">
        Everything You Need
      </h2>
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {everythingYouNeed.map((item, index) => (
          <div
            key={index}
            className="p-4 border border-gray-600 rounded-lg shadow-sm hover:shadow-lg transform hover:scale-105 transition duration-300"
          >
            <div className="flex items-center justify-center w-16 h-16 mx-auto mb-4 rounded-full bg-gradient-to-br">
              {item.icon && (
                <item.icon
                  className={`w-10 h-10 text-green-200 ${item.color}`}
                />
              )}
            </div>
            <h3 className="text-lg font-bold text-gray-300">{item.title}</h3>
            <p className="text-sm text-gray-400">{item.description}</p>
          </div>
        ))}
      </div>
    </section>
  );
};
