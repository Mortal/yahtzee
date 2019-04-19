from yahtzeevalue import Database


if __name__ == "__main__":
    import sys
    with Database(sys.argv[1]) as db:
        print(db[0])
