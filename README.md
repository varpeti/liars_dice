# Liar's Dice 🎲

Bot 🤖 Programming Game

## Basic Rule

- **Face 1 (One) is a joker** - it counts as any number or face when determining if a bid is correct.

## How to Start

- Each player rolls X dice (you can't see other players' dice)
- The first player makes a bid by saying: "I think there are N dice showing face F in the game"

## Taking Your Turn

The next player (the "Caller") has three choices:

### 1. Make a Higher Bid

- You can either:
  - Increase the Number of dice
  - Increase the Face value
  - Or do both
- **Special rule**: You can also "halve the number and call face one" (e.g., from 6 dice showing 3, you can bid 3 dice showing 1)
- **Special rule for face 1 bids**: If you're currently bidding face 1, you must choose:
  - Increase the number but keep face 1
  - Double the number +1 and call any face (e.g., from 4 dice showing 1, you can bid 9 dice showing 2)

### 2. Call the Previous Player a "Liar"

- The round ends immediately
- Count all dice showing the face that was bid (including jokers)
- **If there are fewer dice than bid**: The previous player loses one die
- **If there are equal or more dice than bid**: The Caller loses one die

### 3. Call "Exactly"

- The round ends immediately
- Count all dice showing the face that was bid (including jokers)
- **If the count matches exactly**: The Caller gains one die
- **If the count doesn't match exactly**: The Caller loses one die

## Game End

- The game continues until only one player has dice remaining

## Examples

### Example 1 - Making a Higher Bid

Player A says: "I think there are 3 dice showing 4 in the game"
Player B can respond with:

- "I think there are 4 dice showing 4" (increase number)
- "I think there are 3 dice showing 5" (increase face)
- "I think there are 4 dice showing 5" (increase both)
- "I think there are 2 dice showing 1" (halve number rounded up and call joker)

### Example 1 - Making a Higher Bid on Joker

Player A says: "I think there are 3 dice showing 1 in the game"
Player B can respond with:

- "I think there are 4 dice showing 1" (increase number)
- "I think there are 7 dice showing 4" (double number +1 and call any face)

### Example 2 - Calling a Liar

Player A says: "I think there are 3 dice showing 4 in the game"
Player B says "Liar!"

- Count all dice showing 4 (including jokers)
- If only 2 dice show 4, Player A loses one die
- If 3 or more dice show 4, Player B loses one die

### Example 3 - Calling "Exactly"

Player A says: "I think there are 3 dice showing 4 in the game"
Player B says "Exactly!"

- Count all dice showing 4 (including jokers)
- If exactly 3 dice show 4, Player B gains one die
- If not exactly 3 dice show 4, Player B loses one die
