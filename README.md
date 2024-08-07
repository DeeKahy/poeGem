# poeGem
Temporary buggy website [poeGem](https://poegem.smallapp.cc/). I dont promise it will be up very long. I suggest you just run it locally instead of relying on my bootleg hosting solution.



Welcome to **poeGem**! This tool helps you calculate the most effective gem color to transfigure for marginally better profit in Path of Exile when transfiguring gems.

The accuracy of the results is not guaranteed. After a week of use, it seems to be reasonably effective, but the numbers are kinda jamky. Use at your own risk.

## Code Quality

The codebase is spaghetti code. If you’re up for a challenge, feel free to contribute!

## Future Improvements

I plan to replace Node.js with a more performant, Rust-based solution. In my opinion, JavaScript is better suited for the front end and should not be used in the backend for such purposes.

## How It Works

1. **Data Fetching**: Retrieves gem data from POE Ninja.
2. **Calculation**: Performs basic calculations to estimate gem effectiveness.
3. **Display**: Shows the results, indicating which gem color is most effective for transfiguration.

## Getting Started

1. Clone the repository:
   ```bash
   git clone https://github.com/DeeKahy/poeGem
   ```
2. Navigate to the project directory:
   ```bash
   cd poeGem
   ```
   if you are using nix then you can run `nix develop` to set up the full environmentand installing the dependencies, otherwise continue.
   
3. Install dependencies:
   ```bash
   npm install
   ```
4. Run the tool:
   ```bash
   node app.js
   ```
